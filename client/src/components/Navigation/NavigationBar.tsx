import React from 'react';
import { makeStyles } from '@material-ui/core/styles';
import Paper from '@material-ui/core/Paper';
import Tabs from '@material-ui/core/Tabs';
import Tab from '@material-ui/core/Tab';
import Groups from '../Groups/Groups';
import Transactions from '../Transactions/Transactions';
import Settings from '../Settings/Settings';
import Panels from './Panels';

const useStyles = makeStyles({
  root: {
    flexGrow: 1,
  },        
});

interface IState {
    selectedValue: number;
};

export default class NavigationBar extends React.Component<{}, IState> {
    constructor(props: {}) {
        super(props);
        this.state = {
            selectedValue: 0
        };
    }

    render(): JSX.Element {
        return (
                <Paper>
                    <Tabs
                        value={this.state.selectedValue}
                        onChange={(evt, value: number) => this.setState({ selectedValue: value })}
                        indicatorColor="primary"
                        textColor="primary"
                        centered
                    >
                        <Tab label="Groups" />
                        <Tab label="Transactions" />
                        <Tab label="Settings" />
                    </Tabs>
                    <Panels index={this.state.selectedValue}>
                        <Groups />
                        <Transactions />
                        <Settings />
                    </Panels>
                </Paper>
        );
    }
}