import React from 'react';

interface IProps {
    index: number;
};

export default class Panels extends React.Component<IProps, {}> {
    render(): JSX.Element {
        const props = this.props;

        return <>{React.Children.toArray(props.children)[props.index]}</>
    }
}